use colored::{ColoredString, Colorize};
use hostname::HostnameInfo;

use crate::{config_manager::{color_string, Configuration}, cpu::CPUInfo, memory::MemoryInfo, os::OSInfo, uptime::UptimeInfo};

mod cpu;
mod memory;
mod config_manager;
mod ascii;
mod hostname;
mod os;
mod uptime;

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
    let mut str = String::new();
    let mut title: ColoredString = config_manager::color_string(title, &config.title_color);
    if config.title_bold {
        title = title.bold();
    }
    if config.title_italic {
        title = title.italic();
    }
    str.push_str(&title.to_string());
    str.push_str(&config.seperator);
    str.push_str(&module.format(format, config.decimal_places));
    str
}

fn main() {
    let config: Configuration = config_manager::parse();

    // Since we parse the os-release file in OS anyway, this is always called to get the
    // ascii we want.
    let os: OSInfo = os::get_os();
    let mut ascii: (String, u64) = (String::new(), 0);
    if config.ascii_display {
        ascii = ascii::get_ascii(&os.distro_id);
    }

    let mut line_number: u8 = 0;
    let target_length: u64 = ascii.1;

    let mut split: Vec<&str> = ascii.0.split("\n").collect();
    if split.len() < config.modules.len() {
        // Artificially add length so that all the modules get in
        for _ in 0..(config.modules.len() - split.len()) {
            split.insert(split.len(), "");
        }
    }
    for line in &split {
        // Figure out the color first
        let percentage: f32 = (line_number as f32 / split.len() as f32) as f32;
        // print!("{}", line_number as f32 / split.len() as f32);
        // let percentage: f32 = 1.0;
        // https://stackoverflow.com/a/68457573
        let index: u8 = (((config.ascii_colors.len() - 1) as f32) * percentage).round() as u8;
        // print!(" {}-{}: {} {}", line_number, split.len(), percentage, index);
        let colored = color_string(line, config.ascii_colors.get(index as usize).unwrap());

        // Print the actual ASCII
        print!("{}", colored);
        let remainder = target_length - (line.len() as u64);
        for _ in 0..remainder {
            print!(" ");
        }

        if config.modules.len() > line_number as usize {
            let module: String = config.modules[line_number as usize].to_owned();
            match module.as_str() {
                "hostname" => {
                    let hostname: HostnameInfo = hostname::get_hostname();
                    print!("{}", style_entry(&config.hostname_title, &config.hostname_format, &config, &hostname));
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
                "os" => {
                    print!("{}", style_entry(&config.os_title, &config.os_format, &config, &os));
                }
                "uptime" => {
                    let uptime: UptimeInfo = uptime::get_uptime();
                    print!("{}", style_entry(&config.uptime_title, &config.uptime_format, &config, &uptime));
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
