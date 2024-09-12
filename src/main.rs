use std::time::Duration;
use std::{cmp::max, env, process::exit, time::Instant};

use formatter::CrabFetchColor;
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
use syscalls::SyscallCache;

use crate::ascii::get_ascii_line;
use crate::modules::localip::{self, LocalIPInfo};

mod modules;
mod config_manager;
mod ascii;
mod formatter;
mod proccess_info;
mod versions;
mod package_managers;
mod module;
mod util;
mod syscalls;

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
    /// Enables the inline values option.
    inline_values: bool,

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

// Macro for calling most module types
#[macro_export]
macro_rules! run_generic_module {
    ($mod: ident, $type: ident, $run: ident, $known: expr, $config: expr, $err: expr, $out: expr, $($rargs:tt)*) => {
        if $known.is_none() {
            $known = Some($mod::$run($($rargs)*));
        }
        match $known.as_ref().unwrap() {
            Ok(x) => $out.push(x.style(&$config)),
            Err(e) => {
                if $err {
                    $out.push((String::new(), e.to_string()));
                } else {
                    $out.push($type::unknown_output(&$config));
                }
            },
        }; 
    };
}
#[macro_export]
macro_rules! run_multiline_module {
    ($mod: ident, $type: ident, $run: ident, $known: expr, $config: expr, $err: expr, $out: expr, $($rargs:tt)*) => {
        if $known.is_none() {
            $known = Some($mod::$run($($rargs)*));
        }
        match $known.as_ref().unwrap() {
            Ok(x) => {
                for y in x {
                    $out.push(y.style(&$config));
                }
            },
            Err(e) => {
                if $err {
                    $out.push((String::new(), e.to_string()));
                } else {
                    $out.push($type::unknown_output(&$config));
                }
            },
        }; 
    };
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
    localip: Option<Result<Vec<LocalIPInfo>, ModuleError>>,
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
            localip: None,
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
    let mut config: Configuration = match config_manager::parse(&args.config, &args.module_override, &args.ignore_config_file) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            exit(-1);
        },
    };
    print_bench_time(args.benchmark, args.benchmark_warn, "Parsing Config", bench);

    // if config isn't supprsesing errors, make it go down to args
    let log_errors: bool = { if !config.suppress_errors { !args.suppress_errors } else { !config.suppress_errors } };
    // Inline values
    let inline_values: bool = config.inline_values || args.inline_values;

    // Define our module outputs, and figure out the max title length
    let mut known_outputs: ModuleOutputs = ModuleOutputs::new();
    print_bench_time(args.benchmark, args.benchmark_warn, "Pre-Process", bench);

    // Pre-Process any package manager info we may need
    let bench: Option<Instant> = benchmark_point(args.benchmark);
    let mut package_managers: ManagerInfo = ManagerInfo::new();
    package_managers.probe_and_cache();
    print_bench_time(args.benchmark, args.benchmark_warn, "Cache Package Managers", bench);

    // Setup our syscall cache
    let mut syscall_cache: SyscallCache = SyscallCache::new();

    // Set the title color if we're usign os colors
    if config.use_os_color {
        let c: CrabFetchColor = if let Some(ref x) = args.distro_override {
            formatter::find_os_color(x)
        } else {
            if known_outputs.os.is_none() {
                let os_bench: Option<Instant> = benchmark_point(args.benchmark); 
                known_outputs.os = Some(os::get_os(&config, &mut syscall_cache));
                print_bench_time(args.benchmark, args.benchmark_warn, "OS (for OS Color)", os_bench);
            }
            let mut temp: CrabFetchColor = formatter::find_os_color(&known_outputs.os.as_ref().unwrap().as_ref().unwrap().distro_id);
            if temp == CrabFetchColor::Clear {
                temp = formatter::find_os_color(&known_outputs.os.as_ref().unwrap().as_ref().unwrap().distro_id_like)
            }

            #[cfg(target_os = "linux")]
            if temp == CrabFetchColor::Clear {
                temp = formatter::find_os_color("linux")
            }

            temp
        };

        config.title_color = c.clone();
        if config.ascii.display {
            config.ascii.colors = vec![c];
        }
    }

    // 
    //  Detect
    //
    let detect_bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut output: Vec<(String, String)> = Vec::new();
    let mut cur_segment_length: usize = 0;
    for module in &config.modules {
        let module_parse_bench: Option<Instant> = benchmark_point(args.benchmark); 
        let module_split: Vec<&str> = module.split(':').collect();
        let module_name: &str = module_split[0];
        match module_name {
            "space" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                output.push((String::new(), String::new()));
                print_bench_time(args.benchmark, args.benchmark_warn, "Space Module", bench);
            },
            "underline" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let underline_length: usize = module_split[1].parse().unwrap();
                output.push((String::new(), config.underline_character.to_string().repeat(underline_length)));
                print_bench_time(args.benchmark, args.benchmark_warn, "Underline Module", bench);
            },
            "segment" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let segment_name: &str = module_split[1];  
                let segment_string: String = config.segment_top.replace("{name}", segment_name);
                output.push((String::new(), formatter::replace_color_placeholders(&segment_string, &config)));
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
                output.push((String::new(), formatter::replace_color_placeholders(&segment_string, &config)));
                print_bench_time(args.benchmark, args.benchmark_warn, "Segment End", bench);
            },
            "hostname" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(hostname, HostnameInfo, get_hostname, known_outputs.hostname, config, log_errors, output, &config, &mut syscall_cache);
                print_bench_time(args.benchmark, args.benchmark_warn, "Hostname Module", bench);
            },
            "cpu" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(cpu, CPUInfo, get_cpu, known_outputs.cpu, config, log_errors, output, &config);
                print_bench_time(args.benchmark, args.benchmark_warn, "CPU Module", bench);
            },
            "gpu" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.gpu.is_none() {
                    known_outputs.gpu = Some(gpu::get_gpus(&config));
                }
                match known_outputs.gpu.as_ref().unwrap() {
                    Ok(gpus) => {
                        let mut index: u8 = 1;
                        for gpu in gpus {
                            let mut gpu = gpu.clone();
                            gpu.set_index(index);
                            output.push(gpu.style(&config));
                            index += 1;
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push((String::new(), e.to_string()));
                        } else {
                            output.push(GPUInfo::unknown_output(&config));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "GPU Module", bench);
            },
            "memory" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(memory, MemoryInfo, get_memory, known_outputs.memory, config, log_errors, output, );
                print_bench_time(args.benchmark, args.benchmark_warn, "Memory Module", bench);
            },
            "swap" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(swap, SwapInfo, get_swap, known_outputs.swap, config, log_errors, output, &mut syscall_cache);
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
                            output.push(mount.style(&config))
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push((String::new(), e.to_string()));
                        } else {
                            output.push(MountInfo::unknown_output(&config));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Mounts Module", bench);
            },
            "host" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.host.is_none() {
                    known_outputs.host = Some(host::get_host(&config));
                }
                match known_outputs.host.as_ref().unwrap() {
                    Ok(host) => {
                        output.push(host.style(&config));
                        if config.host.newline_chassis {
                            output.push(host.style_chassis(&config));
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push((String::new(), e.to_string()));
                        } else {
                            output.push(HostInfo::unknown_output(&config));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, args.benchmark_warn, "Host Module", bench);
            },
            "displays" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_multiline_module!(displays, DisplayInfo, get_displays, known_outputs.displays, config, log_errors, output, &config);
                print_bench_time(args.benchmark, args.benchmark_warn, "Displays Module", bench);
            },
            "os" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.os.is_none() {
                    known_outputs.os = Some(os::get_os(&config, &mut syscall_cache));
                }
                match known_outputs.os.as_ref().unwrap() {
                    Ok(os) => {
                        output.push(os.style(&config));
                        if config.os.newline_kernel {
                            output.push(os.style_kernel(&config));
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push((String::new(), e.to_string()));
                        } else {
                            output.push(OSInfo::unknown_output(&config));
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
                output.push(known_outputs.packages.as_ref().unwrap().style(&config));
                print_bench_time(args.benchmark, args.benchmark_warn, "Packages Module", bench);
            },
            "desktop" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(desktop, DesktopInfo, get_desktop, known_outputs.desktop, config, log_errors, output, &config);
                print_bench_time(args.benchmark, args.benchmark_warn, "Desktop Module", bench);
            },
            "terminal" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(terminal, TerminalInfo, get_terminal, known_outputs.terminal, config, log_errors, output, &config, &package_managers);
                print_bench_time(args.benchmark, args.benchmark_warn, "Terminal Module", bench);
            },
            "shell" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(shell, ShellInfo, get_shell, known_outputs.shell, config, log_errors, output, &config, &package_managers);
                print_bench_time(args.benchmark, args.benchmark_warn, "Shell Module", bench);
            },
            "battery" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_multiline_module!(battery, BatteryInfo, get_batteries, known_outputs.battery, config, log_errors, output, );
                print_bench_time(args.benchmark, args.benchmark_warn, "Battery Module", bench);
            },
            "uptime" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(uptime, UptimeInfo, get_uptime, known_outputs.uptime, config, log_errors, output, &mut syscall_cache);
                print_bench_time(args.benchmark, args.benchmark_warn, "Uptime Module", bench);
            },
            "locale" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(locale, LocaleInfo, get_locale, known_outputs.locale, config, log_errors, output, );
                print_bench_time(args.benchmark, args.benchmark_warn, "Locale Module", bench);
            },
            #[cfg(feature = "player")]
            "player" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_multiline_module!(player, PlayerInfo, get_players, known_outputs.player, config, log_errors, output, &config);
                print_bench_time(args.benchmark, args.benchmark_warn, "Player Module", bench);
            },
            "editor" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(editor, EditorInfo, get_editor, known_outputs.editor, config, log_errors, output, &config, &package_managers);
                print_bench_time(args.benchmark, args.benchmark_warn, "Editor Module", bench);
            },
            "initsys" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(initsys, InitSystemInfo, get_init_system, known_outputs.initsys, config, log_errors, output, &config, &package_managers);
                print_bench_time(args.benchmark, args.benchmark_warn, "InitSys Module", bench);
            },
            "processes" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_generic_module!(processes, ProcessesInfo, get_process_count, known_outputs.processes, config, log_errors, output, );
                print_bench_time(args.benchmark, args.benchmark_warn, "Processes Module", bench);
            },
            "datetime" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.datetime.is_none() {
                    known_outputs.datetime = Some(datetime::get_date_time());
                }
                output.push(known_outputs.datetime.as_ref().unwrap().style(&config));
                print_bench_time(args.benchmark, args.benchmark_warn, "Datetime Module", bench);
            },
            "localip" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                run_multiline_module!(localip, LocalIPInfo, get_local_ips, known_outputs.localip, config, log_errors, output, );
                print_bench_time(args.benchmark, args.benchmark_warn, "Local IP Module", bench);
            }

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
                output.push((String::new(), str));
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
                output.push((String::new(), str));
                print_bench_time(args.benchmark, args.benchmark_warn, "Bright Colors Module", bench);
            }
            _ => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if config.unknown_as_text {
                    output.push((String::new(), formatter::replace_color_placeholders(module_name, &config)));
                } else {
                    output.push((String::new(), format!("Unknown module: {}", module_name)));
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
    let mut ascii_split: Vec<&str> = Vec::new();
    let mut ascii_length: usize = 0;
    let mut ascii_target_length: u16 = 0;
    // :(
    let fuck_off_borrow_checker: String;
    if config.ascii.display {
        if known_outputs.os.is_none() {
            let os_bench: Option<Instant> = benchmark_point(args.benchmark); 
            known_outputs.os = Some(os::get_os(&config, &mut syscall_cache));
            print_bench_time(args.benchmark, args.benchmark_warn, "OS (for ASCII)", os_bench);
        }
        if known_outputs.os.as_ref().unwrap().is_ok() {
            // Calculate the ASCII stuff while we're here
            let ascii: (String, u16) = if args.distro_override.is_some() {
                ascii::get_ascii(&args.distro_override.clone().unwrap())
            } else {
                let mut temp: (String, u16) = ascii::get_ascii(&known_outputs.os.as_ref().unwrap().as_ref().unwrap().distro_id);
                if temp.0.is_empty() {
                    temp = ascii::get_ascii(&known_outputs.os.as_ref().unwrap().as_ref().unwrap().distro_id_like);
                }
                #[cfg(target_os = "linux")]
                if temp.0.is_empty() {
                    temp = ascii::get_ascii("linux");
                }

                temp
            };
            fuck_off_borrow_checker = ascii.0;
            ascii_split = fuck_off_borrow_checker.split('\n').filter(|x| x.trim() != "").collect();
            ascii_length = ascii_split.len();
            ascii_target_length = ascii.1 + config.ascii.margin;
        }
    }

    // get the maximum module line length for both right side ascii + inline values
    let mut max_title_len: usize = 0;
    let mut max_total_len: usize = 0;
    // no need to even calculate it if not
    if config.ascii.side == "right" || inline_values {
        for out in &output {
            max_title_len = max(max_title_len, strip_ansi_escapes::strip_str(&out.0).chars().count());
            max_total_len = max(max_total_len, strip_ansi_escapes::strip_str(&out.0).chars().count() + strip_ansi_escapes::strip_str(&out.1).chars().count());
        }
        if inline_values {
            max_total_len += max_title_len;
        }
    }

    print_bench_time(args.benchmark, args.benchmark_warn, "Display ASCII Pre-Calc", ascii_bench);

    // the actual outputs
    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut current_line: usize = 0;
    // top ascii
    if config.ascii.display && config.ascii.side == "top" {
        #[allow(clippy::mut_range_bound)]
        for _ in current_line..ascii_length {
            println!("{}", get_ascii_line(current_line, &ascii_split, &ascii_target_length, &config));
            current_line += 1;
        }
        // Margin
        print!("{}", "\n".repeat(config.ascii.margin as usize));
    }

    // the modules + left/right ascii alongside them
    for out in output {
        // left ascii
        if config.ascii.display && config.ascii.side == "left" {
            print!("{}", get_ascii_line(current_line, &ascii_split, &ascii_target_length, &config));
        }

        let title_len: usize = strip_ansi_escapes::strip_str(&out.0).chars().count();
        let title_len_inline: usize = if inline_values && !out.0.is_empty() {max_title_len - title_len} else {0};
        print!("{}", out.0); // title
        if inline_values && !out.0.is_empty() {
            print!("{}", " ".repeat(title_len_inline));
        }
        print!("{}", out.1); // value

        // right ascii
        if config.ascii.display && config.ascii.side == "right" {
            // This manually adds the margin to the right, as get_ascii_line only does the left
            let line_length_remainder: usize = max_total_len - (title_len + title_len_inline + strip_ansi_escapes::strip_str(&out.1).chars().count());
            print!("{}", " ".repeat(line_length_remainder + config.ascii.margin as usize));
            print!("{}", get_ascii_line(current_line, &ascii_split, &(ascii_target_length - config.ascii.margin), &config));
        }

        current_line += 1;
        println!();
    }
    // bottom ascii
    if config.ascii.display && config.ascii.side == "bottom" {
        // Margin
        print!("{}", "\n".repeat(config.ascii.margin as usize));

        for x in 0..ascii_length {
            println!("{}", get_ascii_line(x, &ascii_split, &ascii_target_length, &config));
        }
    }

    // remaning ascii from left/right in case we're using less modules than required lines
    if current_line < ascii_length && config.ascii.display && (config.ascii.side == "left" || config.ascii.side == "right") {
        let mut ascii_line: usize = current_line;
        for _ in current_line..ascii_length {
            if config.ascii.side == "right" {
                print!("{}", " ".repeat(max_total_len + config.ascii.margin as usize));
            }
            print!("{}", get_ascii_line(ascii_line, &ascii_split, &ascii_target_length, &config));
            ascii_line += 1;
            println!();
        }
    }
    print_bench_time(args.benchmark, args.benchmark_warn, "Module + ASCII Output", bench);

    print_bench_time(args.benchmark, args.benchmark_warn, "Full Runtime of CrabFetch", full_runtime_bench);
}

