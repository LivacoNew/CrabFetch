use core::str;
use std::{fs::File, io::Read, path::Path};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

pub struct CPUInfo {
    name: String,
    cores: u16,
    threads: u16,
    current_clock_mhz: f32,
    max_clock_mhz: f32,
}
#[derive(Deserialize)]
pub struct CPUConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
    pub decimal_places: Option<u32>
}

impl Module for CPUInfo {
    fn new() -> CPUInfo {
        CPUInfo {
            name: "".to_string(),
            cores: 0,
            threads: 0,
            current_clock_mhz: 0.0,
            max_clock_mhz: 0.0,
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.cpu.title_color).is_some() {
            title_color = &CONFIG.cpu.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.cpu.title_bold).is_some() {
            title_bold = CONFIG.cpu.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.cpu.title_italic).is_some() {
            title_italic = CONFIG.cpu.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.cpu.seperator.is_some() {
            seperator = CONFIG.cpu.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.cpu.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        let mut dec_places: u32 = CONFIG.decimal_places;
        if CONFIG.cpu.decimal_places.is_some() {
            dec_places = CONFIG.cpu.decimal_places.unwrap();
        }

        CONFIG.cpu.format.replace("{name}", &self.name)
            .replace("{core_count}", &self.cores.to_string())
            .replace("{thread_count}", &self.threads.to_string())
            .replace("{current_clock_mhz}", &CPUInfo::round(self.current_clock_mhz, dec_places).to_string())
            .replace("{current_clock_ghz}", &CPUInfo::round(self.current_clock_mhz / 1000.0, dec_places).to_string())
            .replace("{max_clock_mhz}", &CPUInfo::round(self.max_clock_mhz, dec_places).to_string())
            .replace("{max_clock_ghz}", &CPUInfo::round(self.max_clock_mhz / 1000.0, dec_places).to_string())
    }
}

pub fn get_cpu() -> CPUInfo {
    let mut cpu: CPUInfo = CPUInfo::new();
    // This ones split into 2 as theres a lot to parse
    get_basic_info(&mut cpu);
    get_max_clock(&mut cpu);

    cpu
}

fn get_basic_info(cpu: &mut CPUInfo) {
    // Starts by reading and parsing /proc/cpuinfo
    // This gives us the cpu name, cores, threads and current clock
    let mut file: File = match File::open("/proc/cpuinfo") {
        Ok(r) => r,
        Err(e) => {
            // Best guess I've got is that we're not on Linux
            // In which case, L
            log_error("CPU", format!("Can't read from /proc/cpuinfo - {}", e));
            return
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("CPU", format!("Can't read from /proc/cpuinfo - {}", e));
            return
        },
    }

    // Now we parse
    // Just doing one entry as the rest are kinda redundant
    let entry: &str = contents.split("\n\n").collect::<Vec<&str>>()[0];
    let lines: Vec<&str> = entry.split("\n").collect();
    let mut cpu_mhz_count: u8 = 0;
    for line in lines {
        if line.starts_with("model name") {
            cpu.name = line.split(": ").collect::<Vec<&str>>()[1].to_string();
        }
        if line.starts_with("cpu cores") {
            cpu.cores = match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("CPU", format!("WARNING: Could not parse cpu cores: {}", e));
                    0
                },
            }
        }
        if line.starts_with("siblings") {
            cpu.threads = match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("CPU", format!("WARNING: Could not parse cpu threads: {}", e));
                    0
                },
            }
        }
        if line.starts_with("cpu MHz") {
            cpu.current_clock_mhz = match line.split(": ").collect::<Vec<&str>>()[1].parse::<f32>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("CPU", format!("WARNING: Could not parse current cpu frequency: {}", e));
                    0.0
                },
            };
            cpu_mhz_count += 1;
        }
    }

    cpu.current_clock_mhz = cpu.current_clock_mhz / cpu_mhz_count as f32;
}
fn get_max_clock(cpu: &mut CPUInfo) {
    // All of this is relative to /sys/devices/system/cpu/cpu0/cpufreq
    // There's 3 possible places to get the frequency in here;
    // - bios_limit - Only present if a limit is set in BIOS
    // - scaling_max_freq - The max freq set by the policy
    // - cpuinfo_max_freq - The max possible the CPU can run at uncapped
    //
    // This just takes the first of those three that are present
    //
    // Source: https://docs.kernel.org/admin-guide/pm/cpufreq.html

    let mut freq_path: Option<&str> = None;
    if Path::new("/sys/devices/system/cpu/cpu0/cpufreq/bios_limit").exists() {
        freq_path = Some("/sys/devices/system/cpu/cpu0/cpufreq/bios_limit");
    } else if Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq").exists() {
        freq_path = Some("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq");
    } else if Path::new("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq").exists() {
        freq_path = Some("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq");
    }

    if freq_path.is_none() {
        // Back up to the repoted value in /proc/cpuinfo
        // At this point all I can assume is your in a VM, which doesn't have any of the above
        // paths and seems to keep a steady CPU frequency in here instead
        // Not the most elegant thing but I can't seem to find anything else to do
        cpu.max_clock_mhz = cpu.current_clock_mhz;
        return
    }

    let mut file: File = match File::open(freq_path.unwrap()) {
        Ok(r) => r,
        Err(e) => {
            log_error("CPU", format!("Can't read from {} - {}", freq_path.unwrap(), e));
            return
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("CPU", format!("Can't read from {} - {}", freq_path.unwrap(), e));
            return
        },
    }

    match contents.trim().parse::<f32>() {
        Ok(r) => {
            cpu.max_clock_mhz = r / 1000.0
        },
        Err(_) => {}
    };
}
