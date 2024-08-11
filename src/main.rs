use std::time::Duration;
use std::{cmp::max, env, process::exit, time::Instant};

use module::{Module, ModuleError};
use modules::battery::{self, BatteryInfo};
use modules::cpu::{self, CPUInfo};
use clap::{ArgAction, Parser};
use colored::Colorize;
use modules::datetime::{self, DateTimeInfo};
use modules::desktop::{self, DesktopInfo};
use modules::displays::{self, DisplayInfo};
use modules::editor::{self, EditorInfo};
use modules::host::{self, HostInfo};
use modules::initsys::{self, InitSystemInfo};
use modules::locale::{self, LocaleInfo};
use modules::memory::{self, MemoryInfo};
use modules::gpu::{self, GPUInfo};
use modules::mounts::{self, MountInfo};
#[cfg(feature = "player")]
use modules::player::{self, PlayerInfo};
use modules::os::{self, OSInfo};
use modules::packages::{self, PackagesInfo};
use modules::processes::{self, ProcessesInfo};
use modules::shell::{self, ShellInfo};
use modules::swap::{self, SwapInfo};
use modules::terminal::{self, TerminalInfo};
use modules::uptime::{self, UptimeInfo};
use modules::hostname::{self, HostnameInfo};
use config_manager::Configuration;
use package_managers::ManagerInfo;

use crate::ascii::get_ascii_line;

mod modules;
mod config_manager;
mod ascii;
mod formatter;
mod proccess_info;
mod versions;
mod package_managers;
mod module;
mod util;

#[derive(Parser)]
#[command(about, long_about = None)]
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

    #[arg(short, long)]
    /// Overrides the distro ASCII to another distro.
    distro_override: Option<String>,

    #[arg(short, long, require_equals(true), default_missing_value("false"), default_value("false"), action=ArgAction::Set)]
    /// Whether to suppress any errors or not.
    suppress_errors: bool,

    #[arg(long)]
    /// Overrides the modules set in your config file. This should be a comma separated list of
    /// modules. E.g cpu,gpu,underline:16,title
    module_override: Option<String>,

    #[arg(long)]
    /// Runs CrabFetch in a "benchmark" mode, showing the total times it takes between each stage
    /// and module detection times.
    benchmark: bool,

    #[arg(long)]
    /// Sets a "warning" color for the benchmark option in µs. If a benchmark time goes above this, 
    /// it will highlight in yellow. If it goes above 1.5x this value, it will output in red.
    benchmark_warn: Option<u128>,

    #[arg(long, short)]
    /// Displays the version of CrabFetch, as well as the current features enabled in this build.
    version: bool
}

// Figures out the max title length for when we're using inline value display
// Some of these require the modules pre-computing as they have dynamic titles
fn calc_max_title_length(config: &Configuration, known_outputs: &mut ModuleOutputs, benchmarking: bool, benchmark_warn: Option<u128>) -> u64 {
    let mut res: u64 = 0;
    // this kinda sucks
    for module in &config.modules {
        match module.as_str() {
            "hostname" => res = max(res, config.hostname.title.chars().count() as u64),
            "cpu" => res = max(res, config.cpu.title.chars().count() as u64),
            "gpu" => res = max(res, {
                if config.gpu.title.contains("{index}") {
                    // Allows for 2 digits in it's place, because no ones going to have more than
                    // 99 GPU's in a single system, and if you are then why tf are you using
                    // CrabFetch of all things go train an AI lmfao
                    return max(res, config.gpu.title.chars().count() as u64 - 5);
                }
                max(res, config.gpu.title.chars().count() as u64)
            }),
            "memory" => res = max(res, config.memory.title.chars().count() as u64),
            "swap" => res = max(res, config.swap.title.chars().count() as u64),
            "mounts" => res = max(res, {
                let bench: Option<Instant> = benchmark_point(benchmarking); 

                if known_outputs.mounts.is_none() {
                    known_outputs.mounts = Some(mounts::get_mounted_drives(config));
                }
                let mut length: u64 = 0;
                if known_outputs.mounts.as_ref().unwrap().is_err() {
                    continue;
                }
                for info in known_outputs.mounts.as_ref().unwrap().as_ref().unwrap() {
                    if info.is_ignored(config) {
                        continue;
                    }
                    length = max(info.get_title_size(config), length);
                }

                print_bench_time(benchmarking, benchmark_warn, "Mounts Module (Pre-comp for max_title_length)", bench);

                length
            }),
            "host" => res = max(res, config.host.title.chars().count() as u64),
            "displays" => res = max(res, {
                let bench: Option<Instant> = benchmark_point(benchmarking); 

                if known_outputs.displays.is_none() {
                    known_outputs.displays = Some(displays::get_displays(config));
                }
                let mut length: u64 = 0;
                if known_outputs.displays.as_ref().unwrap().is_err() {
                    continue;
                }
                for display in known_outputs.displays.as_ref().unwrap().as_ref().unwrap() {
                    length = max(display.get_title_size(config), length);
                }

                print_bench_time(benchmarking, benchmark_warn, "Displays Module (Pre-comp for max_title_length)", bench);

                length
            }),
            "os" => res = max(res, config.os.title.chars().count() as u64),
            "packages" => res = max(res, config.packages.title.chars().count() as u64),
            "desktop" => res = max(res, config.desktop.title.chars().count() as u64),
            "terminal" => res = max(res, config.terminal.title.chars().count() as u64),
            "shell" => res = max(res, config.shell.title.chars().count() as u64),
            "battery" => res = max(res, config.battery.title.chars().count() as u64),
            "uptime" => res = max(res, config.uptime.title.chars().count() as u64),
            "locale" => res = max(res, config.locale.title.chars().count() as u64),
            #[cfg(feature = "player")]
            "player" => res = max(res, config.player.title.chars().count() as u64),
            "editor" => res = max(res, config.editor.title.chars().count() as u64),
            "initsys" => res = max(res, config.initsys.title.chars().count() as u64),
            _ => {}
        }
    }

    res
}
// This is done here simply to make the main function not as indented of a mess, it's abstracted into here
fn benchmark_point(benchmarking: bool) -> Option<Instant> {
    if !benchmarking {return None;}
    Some(Instant::now())
}
fn print_bench_time(benchmarking: bool, benchmark_warn: Option<u128>, name: &str, time: Option<Instant>) {
    if !benchmarking {
        return;
    }
    if time.is_none() {
        return;
    }

    // This is different to module bench times fyi
    let t: Duration = time.unwrap().elapsed();
    let mut t_output: String = format!("{:2?}", t);
    if let Some(threshold) = benchmark_warn {
        if t.as_micros() > ((threshold as f64 * 1.5) as u128) {
            t_output = t_output.bright_red().to_string();
        } else if t.as_micros() > threshold {
            t_output = t_output.bright_yellow().to_string();
        }
    }
    println!("[Benchmark] {}: {}", name, t_output);
}




// Stores all the module's outputs as we know them
// This is to prevent us doing additional work when we don't need to, when modules need shared data
struct ModuleOutputs {
    hostname: Option<Result<HostnameInfo, ModuleError>>,
    cpu: Option<Result<CPUInfo, ModuleError>>,
    gpu: Option<Result<Vec<GPUInfo>, ModuleError>>,
    memory: Option<Result<MemoryInfo, ModuleError>>,
    swap: Option<Result<SwapInfo, ModuleError>>,
    mounts: Option<Result<Vec<MountInfo>, ModuleError>>,
    host: Option<Result<HostInfo, ModuleError>>,
    displays: Option<Result<Vec<DisplayInfo>, ModuleError>>,
    packages: Option<PackagesInfo>,
    desktop: Option<Result<DesktopInfo, ModuleError>>,
    terminal: Option<Result<TerminalInfo, ModuleError>>,
    shell: Option<Result<ShellInfo, ModuleError>>,
    battery: Option<Result<Vec<BatteryInfo>, ModuleError>>,
    uptime: Option<Result<UptimeInfo, ModuleError>>,
    locale: Option<Result<LocaleInfo, ModuleError>>,
    #[cfg(feature = "player")]
    player: Option<Result<Vec<PlayerInfo>, ModuleError>>,
    editor: Option<Result<EditorInfo, ModuleError>>,
    os: Option<Result<OSInfo, ModuleError>>,
    initsys: Option<Result<InitSystemInfo, ModuleError>>,
    processes: Option<Result<ProcessesInfo, ModuleError>>,
    datetime: Option<DateTimeInfo>,

    // Any other potentially duplicated work can be put here and passed into modules
    syscall_sysinfo: Option<libc::sysinfo>,
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
            #[cfg(feature = "player")]
            player: None,
            editor: None,
            os: None,
            initsys: None,
            processes: None,
            datetime: None,
            syscall_sysinfo: None,
        }
    }
}

fn main() {
    let full_runtime_bench: Option<Instant> = benchmark_point(true); // True as it's before any parsing

    // Are we defo in Linux?
    if env::consts::OS != "linux" && env::consts::OS != "android" {
        println!("CrabFetch only supports Linux! If you want to go through and add support for your own OS, make a pull request :)");
        exit(-1);
    }

    // 
    //  Parse 
    //

    // Get the args/config stuff out of the way
    let args_bench: Option<Instant> = benchmark_point(true); // Just true as it's before we parse it
    let args: Args = Args::parse();
    print_bench_time(args.benchmark, args.benchmark_warn, "Args Parsing", args_bench);
    
    if args.version {
        let version: &str = env!("CARGO_PKG_VERSION");
        let hash: &str = env!("GIT_HASH");
        let date: &str = env!("GIT_DATE");
        let message: &str = env!("GIT_MESSAGE");
        let message_lines: Vec<&str> = message.split("<br>").collect();

        println!("CrabFetch {version}");
        println!();
        println!("Built From: {hash} ({date})");
        for line in message_lines {
            println!("  {}", line.trim());
        }
        println!();
        println!("Build contains feature flags:");

        // likely messy 
        #[cfg(feature = "android")]
        println!(" + android");
        #[cfg(not(feature = "android"))]
        println!(" - android");

        #[cfg(feature = "player")]
        println!(" + player");
        #[cfg(not(feature = "player"))]
        println!(" - player");

        #[cfg(feature = "rpm_packages")]
        println!(" + rpm_packages");
        #[cfg(not(feature = "rpm_packages"))]
        println!(" - rpm_packages");

        exit(0);
    }
    if args.generate_config_file {
        let bench: Option<Instant> = benchmark_point(args.benchmark); 
        config_manager::generate_config_file(args.config.clone());
        print_bench_time(args.benchmark, args.benchmark_warn, "Generating Config File", bench);
        exit(0);
    }
    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    let config: Configuration = match config_manager::parse(&args.config, &args.module_override, &args.ignore_config_file) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            exit(-1);
        },
    };
    print_bench_time(args.benchmark, args.benchmark_warn, "Parsing Config", bench);

    // if config isn't supprsesing errors, make it go down to args
    let log_errors: bool = { if !config.suppress_errors { !args.suppress_errors } else { !config.suppress_errors } };

    // Define our module outputs, and figure out the max title length
    let mut known_outputs: ModuleOutputs = ModuleOutputs::new();
    let max_title_length: u64 = calc_max_title_length(&config, &mut known_outputs, args.benchmark, args.benchmark_warn);

    // Pre-Process any package manager info we may need
    let bench: Option<Instant> = benchmark_point(args.benchmark);
    let mut package_managers: ManagerInfo = ManagerInfo::new();
    package_managers.probe_and_cache();
    print_bench_time(args.benchmark, args.benchmark_warn, "Pre-Process and Cache Package Managers", bench);

    // 
    //  Detect
    //
    let detect_bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut output: Vec<String> = Vec::new();
    let mut cur_segment_length: usize = 0;
    for module in &config.modules {
        let module_parse_bench: Option<Instant> = benchmark_point(args.benchmark); 
        let module_split: Vec<&str> = module.split(':').collect();
        let module_name: &str = module_split[0];
        match module_name {
            "space" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                output.push(String::new());
                print_bench_time(args.benchmark, args.benchmark_warn, "Space Module", bench);
            },
            "underline" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let underline_length: usize = module_split[1].parse().unwrap();
                output.push(config.underline_character.to_string().repeat(underline_length));
                print_bench_time(args.benchmark, args.benchmark_warn, "Underline Module", bench);
            },
            "segment" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let segment_name: &str = module_split[1];  
                let segment_string: String = config.segment_top.replace("{name}", segment_name);
                output.push(formatter::replace_color_placeholders(&segment_string));
                cur_segment_length = segment_name.len();
                print_bench_time(args.benchmark, args.benchmark_warn, "Segment Start", bench);
            },
            "end_segment" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 

                let index: usize = config.segment_bottom.find("{name_sized_gap:").unwrap();
                let split: &Vec<char> = &config.segment_bottom[index+16..].chars().collect::<Vec<char>>();
                let char: &char = split.first().unwrap();

                let target = format!("{{name_sized_gap:{}}}", char);
                let segment_string: String = config.segment_bottom.replace(&target, &char.to_string().repeat(cur_segment_length + 2));
                output.push(formatter::replace_color_placeholders(&segment_string));
                print_bench_time(args.benchmark, args.benchmark_warn, "Segment End", bench);
            },
            "hostname" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.hostname.is_none() {
                    known_outputs.hostname = Some(hostname::get_hostname());
                }
                match known_outputs.hostname.as_ref().unwrap() {
                    Ok(hostname) => output.push(hostname.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(HostnameInfo::unknown_output(&config, max_title_length));
                        }
                    },
                };
                print_bench_time(args.benchmark, args.benchmark_warn, "Hostname Module", bench);
            },
            "cpu" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.cpu.is_none() {
                    known_outputs.cpu = Some(cpu::get_cpu(config.cpu.remove_trailing_processor));
                }
                match known_outputs.cpu.as_ref().unwrap() {
                    Ok(cpu) => output.push(cpu.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(CPUInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "CPU Module", bench);
            },
            "gpu" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.gpu.is_none() {
                    known_outputs.gpu = Some(gpu::get_gpus(config.gpu.amd_accuracy, config.gpu.ignore_disabled_gpus));
                }
                match known_outputs.gpu.as_ref().unwrap() {
                    Ok(gpus) => {
                        let mut index: u8 = 1;
                        for gpu in gpus {
                            let mut gpu = gpu.clone();
                            gpu.set_index(index);
                            output.push(gpu.style(&config, max_title_length));
                            index += 1;
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(GPUInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "GPU Module", bench);
            },
            "memory" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.memory.is_none() {
                    known_outputs.memory = Some(memory::get_memory());
                }
                match known_outputs.memory.as_ref().unwrap() {
                    Ok(memory) => output.push(memory.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(MemoryInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Memory Module", bench);
            },
            "swap" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.swap.is_none() {
                    known_outputs.swap = Some(swap::get_swap(&mut known_outputs.syscall_sysinfo));
                }
                match known_outputs.swap.as_ref().unwrap() {
                    Ok(swap) => output.push(swap.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(SwapInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Swap Module", bench);
            },
            "mounts" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.mounts.is_none() {
                    known_outputs.mounts = Some(mounts::get_mounted_drives(&config));
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
                            output.push(MountInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Mounts Module", bench);
            },
            "host" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.host.is_none() {
                    known_outputs.host = Some(host::get_host());
                }
                match known_outputs.host.as_ref().unwrap() {
                    Ok(host) => {
                        output.push(host.style(&config, max_title_length));
                        if config.host.newline_chassis {
                            output.push(host.style_chassis(&config, max_title_length));
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(HostInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Host Module", bench);
            },
            "displays" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.displays.is_none() {
                    known_outputs.displays = Some(displays::get_displays(&config));
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
                print_bench_time(args.benchmark, args.benchmark_warn, "Displays Module", bench);
            },
            "os" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.os.is_none() {
                    known_outputs.os = Some(os::get_os());
                }
                match known_outputs.os.as_ref().unwrap() {
                    Ok(os) => {
                        output.push(os.style(&config, max_title_length));
                        if config.os.newline_kernel {
                            output.push(os.style_kernel(&config, max_title_length));
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(OSInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "OS Module", bench);
            },
            "packages" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.packages.is_none() {
                    known_outputs.packages = Some(packages::get_packages(&package_managers));
                }
                output.push(known_outputs.packages.as_ref().unwrap().style(&config, max_title_length));
                print_bench_time(args.benchmark, args.benchmark_warn, "Packages Module", bench);
            },
            "desktop" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.desktop.is_none() {
                    known_outputs.desktop = Some(desktop::get_desktop());
                }
                match known_outputs.desktop.as_ref().unwrap() {
                    Ok(desktop) => output.push(desktop.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(DesktopInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Desktop Module", bench);
            },
            "terminal" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.terminal.is_none() {
                    known_outputs.terminal = Some(terminal::get_terminal(config.terminal.chase_ssh_pts, config.terminal.format.contains("{version}"), config.use_version_checksums, &package_managers));
                }
                match known_outputs.terminal.as_ref().unwrap() {
                    Ok(terminal) => output.push(terminal.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(TerminalInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Terminal Module", bench);
            },
            "shell" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.shell.is_none() {
                    known_outputs.shell = Some(shell::get_shell(config.shell.show_default_shell, config.shell.format.contains("{version}"), config.use_version_checksums, &package_managers));
                }
                match known_outputs.shell.as_ref().unwrap() {
                    Ok(shell) => output.push(shell.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(ShellInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Shell Module", bench);
            },
            "battery" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.battery.is_none() {
                    known_outputs.battery = Some(battery::get_batteries());
                }
                match known_outputs.battery.as_ref().unwrap() {
                    Ok(batteries) => {
                        for bat in batteries {
                            output.push(bat.style(&config, max_title_length))
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(BatteryInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Battery Module", bench);
            },
            "uptime" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.uptime.is_none() {
                    known_outputs.uptime = Some(uptime::get_uptime(&mut known_outputs.syscall_sysinfo));
                }
                match known_outputs.uptime.as_ref().unwrap() {
                    Ok(uptime) => output.push(uptime.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(UptimeInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Uptime Module", bench);
            },
            "locale" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.locale.is_none() {
                    known_outputs.locale = Some(locale::get_locale());
                }
                match known_outputs.locale.as_ref().unwrap() {
                    Ok(locale) => output.push(locale.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(LocaleInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Locale Module", bench);
            },
            #[cfg(feature = "player")]
            "player" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.player.is_none() {
                    known_outputs.player = Some(player::get_players(&config.player.ignore));
                }
                match known_outputs.player.as_ref().unwrap() {
                    Ok(players) => {
                        for player in players{
                            output.push(player.style(&config, max_title_length));
                        }
                    }
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(PlayerInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Player Module", bench);
            },
            "editor" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.editor.is_none() {
                    known_outputs.editor = Some(editor::get_editor(config.editor.fancy, config.editor.format.contains("{version}"), config.use_version_checksums, &package_managers));
                }
                match known_outputs.editor.as_ref().unwrap() {
                    Ok(editor) => output.push(editor.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(EditorInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Editor Module", bench);
            },
            "initsys" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.initsys.is_none() {
                    known_outputs.initsys = Some(initsys::get_init_system(config.initsys.format.contains("{version}"), config.use_version_checksums, &package_managers));
                }
                match known_outputs.initsys.as_ref().unwrap() {
                    Ok(init) => output.push(init.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(InitSystemInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "InitSys Module", bench);
            },
            "processes" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.processes.is_none() {
                    known_outputs.processes = Some(processes::get_process_count());
                }
                match known_outputs.processes.as_ref().unwrap() {
                    Ok(processes) => output.push(processes.style(&config, max_title_length)),
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(ProcessesInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Processes Module", bench);
            },
            "datetime" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.datetime.is_none() {
                    known_outputs.datetime = Some(datetime::get_date_time());
                }
                output.push(known_outputs.datetime.as_ref().unwrap().style(&config, max_title_length));
                print_bench_time(args.benchmark, args.benchmark_warn, "Datetime Module", bench);
            },

            // i hate what's below as well, don't worry
            "colors" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let char: &str = &config.color_character;
                let gap: &str = &(" ".repeat(config.color_margin as usize));

                let mut str: String = String::new();
                if config.color_use_background {
                    str.push_str(&char.on_black().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_red().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_green().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_yellow().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_blue().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_magenta().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_cyan().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_white().to_string());
                } else {
                    str.push_str(&char.black().to_string());
                    str.push_str(gap);
                    str.push_str(&char.red().to_string());
                    str.push_str(gap);
                    str.push_str(&char.green().to_string());
                    str.push_str(gap);
                    str.push_str(&char.yellow().to_string());
                    str.push_str(gap);
                    str.push_str(&char.blue().to_string());
                    str.push_str(gap);
                    str.push_str(&char.magenta().to_string());
                    str.push_str(gap);
                    str.push_str(&char.cyan().to_string());
                    str.push_str(gap);
                    str.push_str(&char.white().to_string());
                }
                output.push(str);
                print_bench_time(args.benchmark, args.benchmark_warn, "Colors Module", bench);
            }
            "bright_colors" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let char: &str = &config.color_character;
                let gap: &str = &(" ".repeat(config.color_margin as usize));

                let mut str: String = String::new();
                if config.color_use_background {
                    str.push_str(&char.on_bright_black().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_bright_red().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_bright_green().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_bright_yellow().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_bright_blue().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_bright_magenta().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_bright_cyan().to_string());
                    str.push_str(gap);
                    str.push_str(&char.on_bright_white().to_string());
                } else {
                    str.push_str(&char.bright_black().to_string());
                    str.push_str(gap);
                    str.push_str(&char.bright_red().to_string());
                    str.push_str(gap);
                    str.push_str(&char.bright_green().to_string());
                    str.push_str(gap);
                    str.push_str(&char.bright_yellow().to_string());
                    str.push_str(gap);
                    str.push_str(&char.bright_blue().to_string());
                    str.push_str(gap);
                    str.push_str(&char.bright_magenta().to_string());
                    str.push_str(gap);
                    str.push_str(&char.bright_cyan().to_string());
                    str.push_str(gap);
                    str.push_str(&char.bright_white().to_string());
                }
                output.push(str);
                print_bench_time(args.benchmark, args.benchmark_warn, "Bright Colors Module", bench);
            }
            _ => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if config.unknown_as_text {
                    output.push(formatter::replace_color_placeholders(module_name));
                } else {
                    output.push(format!("Unknown module: {}", module_name));
                }
                print_bench_time(args.benchmark, args.benchmark_warn, "Unknown Module / Custom Text", bench);
            }
        }
        print_bench_time(args.benchmark, args.benchmark_warn, "  Entire Module Parse/Detection", module_parse_bench);
    }
    print_bench_time(args.benchmark, args.benchmark_warn, "Entire detection step", detect_bench);


    // 
    //  Display
    //
    let ascii_bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut ascii: (String, u16) = (String::new(), 0);
    if config.ascii.display {
        if known_outputs.os.is_none() {
            let os_bench: Option<Instant> = benchmark_point(args.benchmark); 
            known_outputs.os = Some(os::get_os());
            print_bench_time(args.benchmark, args.benchmark_warn, "OS (for ASCII)", os_bench);
        }
        if known_outputs.os.as_ref().unwrap().is_ok() {
            // Calculate the ASCII stuff while we're here
            if args.distro_override.is_some() {
                ascii = ascii::get_ascii(&args.distro_override.clone().unwrap());
            } else {
                ascii = ascii::get_ascii(&known_outputs.os.as_ref().unwrap().as_ref().unwrap().distro_id);
            }
        }
    }
    let mut ascii_split: Vec<&str> = Vec::new();
    let mut ascii_length: usize = 0;
    let mut ascii_target_length: u16 = 0;
    if config.ascii.display {
        ascii_split = ascii.0.split('\n').filter(|x| x.trim() != "").collect();
        ascii_length = ascii_split.len();
        ascii_target_length = ascii.1 + config.ascii.margin;
    }
    print_bench_time(args.benchmark, args.benchmark_warn, "Display ASCII Pre-Calc", ascii_bench);

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
    print_bench_time(args.benchmark, args.benchmark_warn, "Module + ASCII Output", bench);
    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    if current_line < ascii_length && config.ascii.display {
        let mut ascii_line: usize = current_line;
        for _ in current_line..ascii_length {
            print!("{}", get_ascii_line(ascii_line, &ascii_split, &ascii_target_length, &config));
            ascii_line += 1;
            println!();
        }
    }
    print_bench_time(args.benchmark, args.benchmark_warn, "Remaining ASCII Output", bench);
    print_bench_time(args.benchmark, args.benchmark_warn, "Full Runtime of CrabFetch", full_runtime_bench);
}

